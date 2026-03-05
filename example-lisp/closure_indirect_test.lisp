;; Test indirect closure calls (calling closures stored in variables)
;; This validates DlispValue-based closure ABI
(defun main ()
  (let ((add1 (lambda (x) (+ x 1)))
        (double (lambda (x) (* x 2)))
        (apply-fn (lambda (f x) (f x))))
    
    ;; Direct indirect call
    (print (add1 10))       ;; 11
    (print (double 5))      ;; 10
    
    ;; Closure passed to another closure
    (print (apply-fn add1 100))   ;; 101
    (print (apply-fn double 25))  ;; 50
    
    ;; HOF with closures
    (print (map double '(1 2 3)))      ;; (2 4 6)
    (print (filter (lambda (x) (> x 3)) '(1 2 3 4 5)))  ;; (4 5)
    (print (reduce (lambda (acc x) (+ acc x)) 0 '(10 20 30)))  ;; 60
  ))
