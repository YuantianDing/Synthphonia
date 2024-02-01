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
            (date.month ntDate)
      ))
      (ntBool Bool (
            (int.is0 ntInt)
            (int.is+ ntInt)
            (int.isN ntInt)
      ))
      (ntDate Int (
            (date.parse ntString)
      ))
      (ntList (List String) (
            (str.split ntString ntString)
      ))
      #data.listsubseq.sample:0
))

(constraint (= (f "1 Nov, 2011") "11"))
(constraint (= (f "Dec 2 2234") "12"))
(constraint (= (f "1-Oct-2132") "10"))
(constraint (= (f "8/5/2021") "8"))



(check-synth)