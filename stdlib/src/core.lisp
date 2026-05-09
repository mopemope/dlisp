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
