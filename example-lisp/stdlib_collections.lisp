;; Clojure-flavoured collection helpers from the bundled stdlib.
;; Map prints are avoided: key order is nondeterministic.
(require "core")

(defun main ()
  ;; distinct / frequencies / group-by
  (print (distinct '(1 2 1 3 2 4 3)))
  (print (get (frequencies '(a b a c a)) 'a))
  (print (count (distinct (map (lambda (x) (% x 3)) '(1 2 3 4 5 6)))))
  ;; merge-with / get-in / assoc-in / update-in
  (print (get (merge-with (lambda (x y) (+ x y)) {:a 1 :b 2} {:b 10 :c 3}) :b))
  (print (get-in {:db {:host "localhost" :port 5432}} [:db :port]))
  (print (get-in {:db {:host "localhost"}} [:db :user]))
  (print (get (assoc-in {} [:server :retries] 3) :server))
  (print (update-in {:count 41} [:count] inc))
  ;; partition / interleave
  (print (partition 2 '(1 2 3 4 5)))
  (print (interleave '(1 2 3) '(a b c)))
  (print (interleave '(1 2) '(x))))
