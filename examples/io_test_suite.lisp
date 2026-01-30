(defun main ()
  (print "--- Test 1: Read existing file (Cargo.toml) ---")
  (let ((content (read-file "Cargo.toml")))
    (if content
        (print "Success: File read")
        (print "Failed: Content is nil")))
  
  (print "\n--- Test 2: Read non-existent file ---")
  (let ((content (read-file "non_existent_file_xyz.txt")))
    (if (= content nil)
        (print "Success: Content is nil as expected")
        (print "Failed: Content should be nil")))
  
  (print "\n--- Test 3: Read this script itself ---")
  (print (read-file "examples/io_test_suite.lisp")))
