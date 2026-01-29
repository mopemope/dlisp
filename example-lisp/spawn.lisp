
(defun task1 ()
    (print "Task 1 started")
    (sleep 1000)
    (print "Task 1 finished"))

(defun task2 ()
    (print "Task 2 started")
    (sleep 2000)
    (print "Task 2 finished"))

(defun main ()
    (print "Main starting tasks...")
    (spawn task1)
    (spawn task2)
    (print "Main spawned tasks, waiting...")
    (sleep 3000)
    (print "Main done."))
