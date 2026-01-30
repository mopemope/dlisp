;;; macros.lisp - Examples of Macros in DLisp

;; 1. 'when' macro
;; Executes body only if condition is true.
;; (when (> 10 5) (print 1) (print 2)) -> (if (> 10 5) (progn (print 1) (print 2)) nil)
;; Note: Since we don't have 'progn' yet, we keep it simple or assume implicit progn in function/macro bodies if supported, 
;; but currently 'if' only takes single expressions. 
;; So we can list them or use a 'do' block if we validize one.
;; For now, let's just do a single expression for simplicity in this demo, or use a list.

(defmacro when (cond body)
  (list 'if cond body 'nil))

(print "--- Testing 'when' macro ---")
(when (> 10 5)
  (print "10 is greater than 5!"))

(when (< 10 5)
  (print "This should NOT print."))


;; 2. 'unless' macro
;; Executes body only if condition is false.
;; (unless (= 1 2) (print "Not equal")) -> (if (= 1 2) nil (print "Not equal"))

(defmacro unless (cond body)
  (list 'if cond 'nil body))

(print "--- Testing 'unless' macro ---")
(unless (= 1 2)
  (print "1 is NOT equal to 2!"))

(unless (= 1 1)
  (print "This should NOT print."))


;; 3. 'incf' style macro (simple version)
;; (inc val) -> (+ val 1)
;; Note: real incf modifies a variable proper, but since we don't have 'setq' or mutable bindings easily accessible here without 'set',
;; we will just make it return the incremented value.

(defmacro inc-expr (x)
  (list '+ x 1))

(print "--- Testing 'inc-expr' macro ---")
(print (inc-expr 99)) ;; Expect 100


;; 4. Nested usage
(print "--- Testing nested macros ---")
(when (> (inc-expr 4) 4)
  (print "Recursive macro expansion works!"))

