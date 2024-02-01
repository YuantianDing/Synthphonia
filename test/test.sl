(set-logic SLIA)

(synth-fun f ((name String)) String
    (
      (Start String (ntString))
      (ntString String (" " "-" "x" "NULL" name
            (str.++ ntString ntString) 
            (str.head ntString ntInt #cost:2)
            (str.tail ntString ntInt #cost:2)
            (list.at ntList ntInt) 
            (int.to.str ntInt #cost:2)
            (str.join ntList ntString)
            (str.retainLl ntString #cost:4)
            (str.retainLc ntString #cost:4)
            (str.retainL ntString #cost:4)
            (str.retainN ntString #cost:4)
            (str.retainLN ntString #cost:4)
            (str.uppercase ntString #cost:4)
            (str.lowercase ntString #cost:4)
            (ite ntBool ntString ntString)
      ))
      (ntInt Int (-1 1 2 3
            (+ ntInt ntInt)
            (int.neg ntInt)
            (str.to.int ntString)
            (list.len ntString)
            (str.count ntString ntString)
      ))
      (ntBool Bool (
            (int.is0 ntInt)
            (int.is+ ntInt)
            (int.isN ntInt)
      ))
      (ntList (List String) (
            (str.split ntString ntString)
      ))
      #data.listsubseq.sample:0
))

(constraint (= (f "875-259-4922") "NULL"))
(constraint (= (f "490-896-3889") "NULL"))
(constraint (= (f "596-501-4296 x0339") "x0339"))
(constraint (= (f "712-973-4124 x6418") "x6418"))
(constraint (= (f "786-628-8081 x8294") "x8294"))
(constraint (= (f "781-771-9145") "NULL"))



(check-synth)