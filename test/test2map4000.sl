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

            (str.retainLl ntString #cost:4)
            (str.retainLc ntString #cost:4)
            (str.retainL ntString #cost:4)
            (str.retainN ntString #cost:4)
            (str.retainLN ntString #cost:4)
            (str.uppercase ntString #cost:4)
            (str.lowercase ntString #cost:4)

            (ite ntBool ntString ntString)
      ) )
      (ntInt Int (-1 1 2 3 4 5
            (+ ntInt ntInt #cost:4)
            (int.neg ntInt)
            (list.len ntString)
            (str.count ntString ntString #cost:2)
            (str.to.int ntString #cost:2)
      ))
      (ntBool Bool (
            (int.is0 ntInt)
            (int.is+ ntInt)
            (int.isN ntInt)
      ))
      (ntList (List String) (
            (str.split ntString ntString)
            (list.map ntList)
      ))
      #data.listsubseq.sample:0
      #increase_cost_limit:4000
))


(constraint (= (f "4,8,6,3,2") "5,9,7,4,3"))


(check-synth)