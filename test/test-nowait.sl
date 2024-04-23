(set-logic SLIA)

(synth-fun f ((name String)) String
    (
      (Start String (ntString))
      (ntString String (" " name
            (str.++ ntString ntString) 
            (str.head ntString ntFloat #cost:4)
            (str.tail ntString ntFloat #cost:4)

            (list.at ntList ntFloat) 
            (str.join ntList ntString)

            (float.fmt ntFloat)
            (int.fmt ntInt)
            (month.fmt ntInt)
            (weekday.fmt ntInt)
            (time.fmt ntTime)

            (str.retainLl ntString #cost:4)
            (str.retainLc ntString #cost:4)
            (str.retainL ntString #cost:4)
            (str.retainN ntString #cost:4)
            (str.retainLN ntString #cost:4)
            (str.uppercase ntString #cost:4)
            (str.lowercase ntString #cost:4)

            (ite ntBool ntString ntString)
      ) )
      (ntInt Int (-1 1 2 4
            (+ ntInt ntInt)
            (int.neg ntInt)

            (date.month ntDate)
            (date.day ntDate)
            (date.year ntDate)
      ))
      (ntFloat Float (-1.0 0.0 1.0 2.0 5.0
            (list.flen ntString)
            (str.fcount ntString ntString)
            (str.to.float ntString)
            (float.+ ntFloat ntFloat)
            (float.neg ntFloat)
            (float.shl10 ntFloat ntInt)
            (float.floor ntFloat ntFloat #cost:2)
            (float.ceil ntFloat ntFloat #cost:2)
            (float.round ntFloat ntFloat #cost:2)
      ))
      (ntDate Int (
            (date.parse ntString)
      ))
      (ntTime Int (15 30 60 3600
            (time.parse ntString)
            (time.floor ntTime ntTime)
            (time.* ntTime ntInt)
      ))
      (ntBool Bool (
            (float.is0 ntFloat #cost:2)
            (float.is+ ntFloat)
            (float.not- ntFloat)
      ))
      (ntList (List String) (
            (str.split ntString ntString)
      ))
      #data.listsubseq.sample:0
      #increase_cost_limit:1000
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