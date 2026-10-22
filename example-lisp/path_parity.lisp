; Shared interpreter/JIT/AOT fixture. Keep the call graph explicit.
(defun path-probe (n)
  (try
    (if (< n 0)
        (throw :neg)
        (+ n (nth (vector 7 8 9) 1)))
    (catch e (error-value e))))

(defun main ()
  (print (path-probe 3))
  (print (path-probe -1)))
