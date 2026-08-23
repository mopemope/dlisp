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
