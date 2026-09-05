;; try/throw on the compiled path: catches, propagation, and neutral values.
;; Golden parity: interpreter and AOT output must be byte-exact.

(defun thrower (v)
  (throw v))

(defun safe-call (f arg)
  (try (f arg) (catch e (string-append "caught: " (str (error-value e))))))

(defun main ()
  ;; Basic catch with error unwrapping.
  (print (try (thrower 42) (catch e (error-value e))))
  ;; No-throw path returns the body value.
  (print (try (* 2 21) (catch e :never)))
  ;; Throw propagates across function boundaries.
  (print (safe-call thrower "deep"))
  ;; Thrown value keeps its type; error? / type-of see the marker.
  (print (type-of (try (thrower :kw) (catch e e))))
  (print (error? (try (thrower "x") (catch e e))))
  ;; Arithmetic on the unwrapped error value.
  (print (try (thrower 99) (catch e (+ (error-value e) 1))))
  ;; Rethrow from an inner catch reaches the outer catch.
  (print
    (try
      (try (thrower :inner) (catch e (throw (error-value e))))
      (catch e2 (error-value e2))))
  ;; Expressions after a throw never run.
  (print (try (throw :after) (print "unreachable") (catch e (error-value e))))
  ;; Throw inside a lambda escapes the lambda into the caller's try.
  (let ((g (lambda () (throw :from-lambda))))
    (print (try (g) (catch e (error-value e)))))
  ;; A catch-less try escapes to the enclosing try, not out of the function.
  (print (try (try (thrower 7)) (catch e (error-value e))))
  (print (try (try (thrower :rethrown)) (catch e (error-value e))))
  ;; Degenerate try forms evaluate to nil / pass through.
  (print (try (+ 1 2)))
  ;; try/throw interacting with while loops.
  (let ((i 0) (caught 0))
    (while (< i 10)
      (try (if (= (% i 2) 0) (throw :even) i) (catch e (setq caught (+ caught 1))))
      (setq i (+ i 1)))
    (print caught))
  ;; The last expression's value becomes the exit status payload.
  (try (thrower :done) (catch e (error-value e))))
