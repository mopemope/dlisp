(defun main ()
  (let ((l '(1 2 3 4 5))
        (f1 (lambda (x) (* x 2)))
        (f2 (lambda (x) (> x 2)))
        (f3 (lambda (acc x) (+ acc x))))
    
    (print "--- Testing map ---")
    (print (map f1 l))       ;; (2 4 6 8 10)
    
    (print "--- Testing filter ---")
    (print (filter f2 l))    ;; (3 4 5)
    
    (print "--- Testing reduce ---")
    (print (reduce f3 0 l))  ;; 15
  ))
