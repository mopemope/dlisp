
(defun fib (n)
    (if (< n 2)
        n
        (+ (fib (- n 1)) (fib (- n 2)))))

(defun main ()
    (print (fib 10)))
