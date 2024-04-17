(define (map ls fn)
  (if (null? ls) 
    () ; cooool!
    (cons (fn (car ls))
          (map (cdr ls) fn))))

(define l1 (list 1 2 3 4)) ;comment here!
; we got comment
(define l2 (cons 5 (cdr (cdr l1))))
(define m1 (map l2 (lambda (x) (* x 2)))) ;end w a comment :)

