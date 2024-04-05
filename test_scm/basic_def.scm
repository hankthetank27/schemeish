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

(/ 10 2 2)
(- 10 5 4)
(= 2 2 2)
(= 1 2 2)

(define (test-if x y)
  (if (= x y)
    "nice!"
    "not nice :/"))
(test-if 1 2)
(test-if 1 1)


