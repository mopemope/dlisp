;; Channels and atoms: deterministic output on every execution path.
;;
;; All channel traffic stays inside `main` so the spawn/recv pairs are
;; always lowered together: compiled code blocks on recv while spawned
;; producers run on OS threads, and the interpreted path polls
;; cooperatively. Mixing an interpreted `(spawn ...)` with a recv inside a
;; separately JIT-compiled function can deadlock the single-threaded
;; executor, so keep them in the same unit.
(require "core")

(defun producer (ch n)
  (send ch (* n n)))

(defun bump (a v)
  (reset! a (+ (deref a) v)))

(defun main ()
  (let ((ch (chan)) (total (atom 0)))
    (spawn producer ch 2)
    (spawn producer ch 3)
    (spawn producer ch 4)
    (bump total (recv ch))
    (bump total (recv ch))
    (bump total (recv ch))
    (close ch)
    ;; closed channel: sends fail, receives drain then return nil
    (print (deref total))
    (print (try-recv ch))
    (print (recv ch))
    (print (send ch 1))))
