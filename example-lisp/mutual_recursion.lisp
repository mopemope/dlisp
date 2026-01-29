
(defun is_even (n)
    (if (= n 0)
        1
        (is_odd (- n 1))))

(defun is_odd (n)
    (if (= n 0)
        0
        (is_even (- n 1))))

(defun main ()
    (print "Is 10 even?")
    (print (is_even 10))
    (print "Is 11 even?")
    (print (is_even 11)))
