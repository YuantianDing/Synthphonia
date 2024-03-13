(set-logic SLIA)

(define-fun f ( (_arg_0 String)) String (str.substr (str.replace _arg_0 "/n" " ") (ite (str.contains (str.replace _arg_0 "/n" "") "/n") (str.indexof (str.substr _arg_0 1 (str.len _arg_0)) (str.substr _arg_0 0 (str.len "/n")) 0) (+ (str.indexof _arg_0 "/n" 0) 1)) (str.len _arg_0)))

(assert (= (f "2/1/2015 - First call/n12/3/2015-order placed/n11/15/2015-follow-up,interested") "11/15/2015-follow-up,interested"))
(assert (= (f "11/1/2015 - First call/n12/3/2015-order placed") "12/3/2015-order placed"))
(assert (= (f "11/1/2015 - First call") "11/1/2015 - First call"))

(check-sat)
