;; Pattern matching via (require "core")
(require "core")

(defun classify (x)
  (match x
    (42 "the answer")
    (:ok "fine")
    ("hi" "greeting")
    (_ "other")))

(defun describe-number (n)
  (match n
    ((m :when (and (number? m) (< m 0))) "negative")
    (0 "zero")
    ((m :when (and (number? m) (> m 100))) "big")
    (m "moderate")))

(defun point-sum (p)
  (match p
    ([x y] (+ x y))
    ([x y &rest more] (+ x y (count more)))))

(defun head-tail (xs)
  (match xs
    (nil "empty")
    ([only] (list :single only))
    ([a b &rest rest] (list :many a b rest))))

(defun read-point (m)
  (match m
    ({:name n} n)
    ({:x x :y y} (+ x y))))

(defun nested (v)
  (match v
    ([{:kind k} val] (list k val))
    (other other)))

(defun guard-fallthrough (n)
  (match n
    ((x :when (and (number? x) (> x 10))) :too-big)
    ((x :when (and (number? x) (> x 5))) :medium)
    (x :small)))

(defun main ()
  ;; literals + wildcard
  (print (classify 42))
  (print (classify :ok))
  (print (classify "hi"))
  (print (classify 'sym))
  ;; guards, including fallthrough to later clauses
  (print (describe-number -3))
  (print (describe-number 0))
  (print (describe-number 500))
  (print (describe-number 7))
  (print (guard-fallthrough 99))
  (print (guard-fallthrough 8))
  (print (guard-fallthrough 2))
  ;; sequence patterns
  (print (point-sum [3 4]))
  (print (point-sum (list 1 2)))
  (print (head-tail nil))
  (print (head-tail [9]))
  (print (head-tail '(1 2 3 4)))
  ;; map patterns
  (print (read-point {:name "dlisp"}))
  (print (read-point {:x 2 :y 3}))
  ;; nested patterns
  (print (nested [{:kind :add} 5]))
  ;; no matching clause yields nil
  (print (match :nope
           (1 :one)
           ([a b] :pair))))
