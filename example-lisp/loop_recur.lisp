;; loop/recur: stack-safe iteration on the interpreter and compiled paths
(defun sum-to (n)
  (loop [k n acc 0]
    (if (= k 0)
        acc
        (recur (- k 1) (+ acc k)))))

(defun fib-iter (n)
  (loop [a 0 b 1 i 0]
    (if (= i n)
        a
        (recur b (+ a b) (+ i 1)))))

(defun coll-build (n)
  (loop [k n acc nil]
    (if (= k 0)
        acc
        (recur (- k 1) (cons k acc)))))

(defun main ()
  (print (sum-to 100))
  (print (fib-iter 20))
  (print (coll-build 5))
  (print (count (coll-build 100000))))
