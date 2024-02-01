(
    (defun power (lambda (x n) (
        if (= n 0)
            1
            (* x (power x (- n 1)))
    )))
    (print (power 2 -1))
    (print (power 2 1))
    (print (power 2 2))
    (print (power 2 3))
    (print (power 2 4))
)