
(defun factorial (n)
    (if (= n 0)
        1
        (* n (factorial (- n 1)))))

(defun main ()
    (print (factorial 5))
    (print (factorial 10)))
