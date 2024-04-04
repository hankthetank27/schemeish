(define (add-with-ten x y)
  (define (add b c)
    (+ 10 b c))
  (+ (add x x) (add y y)))
(- (add-with-ten 1 1) 5 5)

    (((lambda (x)
        (lambda (y)
          (+ x y)))
      3)
     4)

