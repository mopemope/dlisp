(defun inc (x)
  (+ x 1))

(defun dec (x)
  (- x 1))

(defun identity (x)
  x)

(defun constantly (x)
  (lambda (&rest args) x))

(defun second (xs)
  (car (cdr xs)))

(defun third (xs)
  (car (cdr (cdr xs))))

(defun zero? (x)
  (= x 0))

(defun positive? (x)
  (> x 0))

(defun negative? (x)
  (< x 0))

(defun empty-list? (xs)
  (= (count xs) 0))

(defmacro when-let (binding body)
  (let ((name (car binding))
        (expr (second binding)))
    `(let ((,name ,expr))
       (if ,name ,body nil))))

(defun thread-first-step (form acc)
  (if (list? form)
      (cons (car form) (cons acc (cdr form)))
      (list form acc)))

(defun thread-last-step (form acc)
  (if (list? form)
      (append form (list acc))
      (list form acc)))

(defmacro -> (x &rest forms)
  (if (empty-list? forms)
      x
      `(-> ,(thread-first-step (car forms) x)
           ,@(cdr forms))))

(defmacro ->> (x &rest forms)
  (if (empty-list? forms)
      x
      `(->> ,(thread-last-step (car forms) x)
            ,@(cdr forms))))

(defmacro as-> (expr name &rest forms)
  (if (empty-list? forms)
      expr
      `(let ((,name ,expr))
         (as-> ,(car forms) ,name ,@(cdr forms)))))

(defmacro some-> (x &rest forms)
  (if (empty-list? forms)
      x
      (let ((g (gensym)))
        `(let ((,g ,x))
           (if (nil? ,g)
               nil
               (some-> ,(thread-first-step (car forms) g) ,@(cdr forms)))))))

;; Iterative range builder: collects [start,end) stepping by step,
;; prepending to acc and reversing once at the end.
(defun range-iter (start end step)
  (let ((acc nil)
        (curr start))
    (while (if (> step 0) (< curr end) (> curr end))
      (setq acc (cons curr acc))
      (setq curr (+ curr step)))
    (reverse acc)))

;; Validates arguments: integers only, non-zero step. Yields nil otherwise.
(defun range-build (start end step)
  (if (and (= (type-of start) "integer")
           (= (type-of end) "integer")
           (= (type-of step) "integer")
           (not (= step 0)))
      (range-iter start end step)
      nil))

;; (range 5) => (0 1 2 3 4)
;; (range 1 4) => (1 2 3)
;; (range 10 0 -3) => (10 7 4 1)
;; Helpers are defined above so every defun compiles eagerly on the JIT path.
(defun range (&rest args)
  (let ((argc (count args)))
    (cond ((= argc 1) (range-build 0 (first args) 1))
          ((= argc 2) (range-build (first args) (second args) 1))
          ((= argc 3) (range-build (first args) (second args) (third args)))
          (true nil))))

;; ---------------------------------------------------------------------------
;; Collection helpers (Clojure-flavoured; compiled on every path)
;; ---------------------------------------------------------------------------

;; Truthy when x occurs in xs (`=` equality).
(defun member? (x xs)
  (some (lambda (y) (= y x)) xs))

;; xs without duplicates, preserving first-seen order.
(defun distinct (xs)
  (reverse
   (reduce (lambda (acc x)
             (if (member? x acc) acc (cons x acc)))
           nil
           xs)))

;; Map of element -> occurrence count.
(defun frequencies (xs)
  (reduce (lambda (acc x)
            (assoc acc x (+ 1 (get acc x 0))))
          (hash-map)
          xs))

;; Map of (f x) -> list of x's sharing that key.
(defun group-by (f xs)
  (reduce (lambda (acc x)
            (let ((k (f x)))
              (assoc acc k (append (get acc k nil) (list x)))))
          (hash-map)
          xs))

;; Merge two maps; conflicting keys are combined with (f a-val b-val).
;; `contains?` has no codegen lowering, so presence is probed with a
;; sentinel default instead.
(defun merge-with (f a b)
  (reduce (lambda (acc k)
            (let ((cur (get acc k :dlisp-missing)))
              (if (= cur :dlisp-missing)
                  (assoc acc k (get b k))
                  (assoc acc k (f cur (get b k))))))
          a
          (keys b)))

;; Internal: walk the path vector positionally instead of slicing it.
(defun get-in-at (m ks i)
  (if (= i (count ks))
      m
      (let ((v (get m (nth ks i) :dlisp-missing)))
        (if (= v :dlisp-missing)
            nil
            (get-in-at v ks (+ i 1))))))

;; Nested lookup: (get-in user [:address :city]); a missing key yields nil.
(defun get-in (m ks)
  (get-in-at m ks 0))

;; Internal: rebuild nested maps along the path, setting leaf to v.
(defun assoc-in-at (m ks i v)
  (let ((k (nth ks i)))
    (if (= (+ i 1) (count ks))
        (assoc m k v)
        (assoc m k (assoc-in-at (get m k (hash-map)) ks (+ i 1) v)))))

;; Nested association: (assoc-in cfg [:db :host] "localhost")
(defun assoc-in (m ks v)
  (assoc-in-at m ks 0 v))

;; Internal: apply f to the value at the path and store it back.
(defun update-in-at (m ks i f)
  (let ((k (nth ks i)))
    (if (= (+ i 1) (count ks))
        (assoc m k (f (get m k)))
        (assoc m k (update-in-at (get m k (hash-map)) ks (+ i 1) f)))))

;; Nested update with a unary function: (update-in cfg [:retries] inc)
(defun update-in (m ks f)
  (update-in-at m ks 0 f))

;; Consecutive n-element chunks of coll (last chunk may be shorter).
(defun partition (n coll)
  (loop [rest coll acc nil]
    (if (empty? rest)
        (reverse acc)
        (recur (drop n rest) (cons (take n rest) acc)))))

;; Alternating elements of two collections until either runs out.
(defun interleave (a b)
  (reverse
   (loop [ra a rb b acc nil]
     (if (or (empty? ra) (empty? rb))
         acc
         (recur (rest ra) (rest rb)
                (cons (first rb) (cons (first ra) acc)))))))
