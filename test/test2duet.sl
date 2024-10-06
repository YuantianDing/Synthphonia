(set-logic SLIA)

(synth-fun f ((name String)) String
    (
      (Start String (ntString))
      (ntString String ("" name
            (str.++ ntString ntString) 
            (str.head ntString ntInt #cost:4)
            (str.tail ntString ntInt #cost:4)

            (list.at ntList ntInt) 
            (str.join ntList ntString)
            (int.to.str ntInt #cost:2)

            (str.retainLl ntString #cost:3)
            (str.retainLc ntString #cost:3)
            (str.retainL ntString #cost:3)
            (str.retainN ntString #cost:3)
            (str.retainLN ntString #cost:3)
            (str.uppercase ntString #cost:4)
            (str.lowercase ntString #cost:4)

            (ite ntBool ntString ntString)
      ) )
      (ntInt Int (-1 1 2 3 4
            (+ ntInt ntInt)
            (int.neg ntInt)
            (list.len ntString)
            (str.count ntString ntString)
            (str.to.int ntString #cost:2)
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


(constraint (= (f "163") "160-169"))
(constraint (= (f "111") "110-119"))
(constraint (= (f "111") "110-119"))
(constraint (= (f "88") "80-89"))
(constraint (= (f "54") "50-59"))
(constraint (= (f "93") "90-99"))
(constraint (= (f "93") "90-99"))
(constraint (= (f "6") "0-9"))
(constraint (= (f "199") "190-209"))
(constraint (= (f "62") "60-69"))
(constraint (= (f "169") "160-179"))
(constraint (= (f "6") "0-9"))
(constraint (= (f "105") "100-109"))
(constraint (= (f "137") "130-139"))
(constraint (= (f "16") "10-19"))
(constraint (= (f "90") "90-99"))
(constraint (= (f "197") "190-199"))
(constraint (= (f "152") "150-159"))
(constraint (= (f "76") "70-79"))
(constraint (= (f "191") "190-199"))

(check-synth)