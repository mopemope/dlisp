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
