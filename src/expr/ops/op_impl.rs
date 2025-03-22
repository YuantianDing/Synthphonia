
#[macro_export]
macro_rules! impl_basic {
    ($s:ident, $name:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $s(pub usize);
        impl $s {
            pub fn from_config(config: &$crate::parser::config::Config) -> Self {
                Self(config.get_usize("cost").unwrap_or(1))
            }
            pub fn name() -> &'static str {$name}
        }
        impl std::fmt::Display for $s {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Self::name().fmt(f)
            }
        }
        impl Default for $s {
            fn default() -> Self {
                Self::from_config(&Default::default())
            }
        }
    };
}

#[macro_export]
macro_rules! default_value {
    (Str) => { "" }; 
    (Int) => { 0i64 }; 
    (Bool) => { false }; 
    (Float) => { $crate::utils::F64(0.0) }; 
    (ListStr) => { &[] as &[&str] }; 
    (ListInt) => { [] }; 
}
#[macro_export]
macro_rules! impl_name {
    ($s:ident, $name:expr) => {
        impl $s {
            pub fn name() -> &'static str {$name}
        }
        impl std::fmt::Display for $s {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Self::name().fmt(f)
            }
        }
        impl Default for $s {
            fn default() -> Self {
                Self::from_config(&Default::default())
            }
        }
    };
}


#[macro_export]
macro_rules! impl_op1 {
    ($s:ident, $name:expr, $($t1:ident -> $rt:ident { $f:expr }),*) => {
        impl $crate::expr::ops::Op1 for $s {
            fn cost(&self) -> usize { self.0 }
            fn try_eval(&self, a1 : $crate::value::Value) -> (bool, $crate::value::Value) {
                match a1 {
                    $(
                        crate::value::Value::$t1(s) => (true, crate::value::Value::$rt(s.iter().map($f).galloc_scollect())),
                    )*
                    _ => (false, crate::value::Value::Null),
                }
            }
        }
    }
}
#[macro_export]
macro_rules! new_op1 {
    ($s:ident, $name:expr, $($t1:ident -> $rt:ident { $f:expr }),*) => {
        $crate::impl_basic!($s, $name);
        impl $crate::forward::enumeration::Enumerator1 for $s {}
        $crate::impl_op1!($s, $name, $($t1 -> $rt { $f }),*);
    };
}

#[macro_export]
macro_rules! impl_op1_opt {
    ($s:ident, $name:expr, $($t1:ident -> $rt:ident { $f:expr }),*) => {
        impl $crate::expr::ops::Op1 for $s {
            fn cost(&self) -> usize { self.0 }
            fn try_eval(&self, a1 : $crate::value::Value) -> (bool, $crate::value::Value) {
                match a1 {
                    $(
                        crate::value::Value::$t1(s1) => {
                            let mut flag = true;
                            let v = s1.iter().map($f).map(|f| { flag &= f.is_some(); f.unwrap_or($crate::default_value![$rt]) }).galloc_scollect();
                            (flag, crate::value::Value::$rt(v))
                        }
                    )*
                    _ => (false, crate::value::Value::Null),
                }
            }
        }
    };
}
#[macro_export]
macro_rules! new_op1_opt {
    ($s:ident, $name:expr, $($t1:ident -> $rt:ident { $f:expr }),*) => {
        $crate::impl_basic!($s, $name);
        impl $crate::forward::enumeration::Enumerator1 for $s {}
        $crate::impl_op1_opt!($s, $name, $($t1 -> $rt { $f }),*);
    };
}

#[macro_export]
macro_rules! new_op2 {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident) -> $rt:ident { $f:expr }),*) => {
        $crate::impl_basic!($s, $name);
        impl $crate::forward::enumeration::Enumerator2 for $s {}
        $crate::impl_op2!($s, $name, $(($t1, $t2) -> $rt { $f }),*);
    };
}

#[macro_export]
macro_rules! new_op2_opt {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident) -> $rt:ident { $f:expr }),*) => {
        $crate::impl_basic!($s, $name);
        impl $crate::forward::enumeration::Enumerator2 for $s {}
        $crate::impl_op2_opt!($s, $name, $(($t1, $t2) -> $rt { $f }),*);
    };
}

#[macro_export]
macro_rules! new_op3 {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident, $t3:ident) -> $rt:ident { $f:expr }),*) => {
        $crate::impl_basic!($s, $name);
        impl $crate::forward::enumeration::Enumerator3 for $s {}
        $crate::impl_op3!($s, $name, $(($t1, $t2, $t3) -> $rt { $f }),*);
    };
}

#[macro_export]
macro_rules! new_op3_opt {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident, $t3:ident) -> $rt:ident { $f:expr }),*) => {
        $crate::impl_basic!($s, $name);
        impl $crate::forward::enumeration::Enumerator3 for $s {}
        $crate::impl_op3_opt!($s, $name, $(($t1, $t2, $t3) -> $rt { $f }),*);
    };
}
#[macro_export]
macro_rules! impl_op2 {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident) -> $rt:ident { $f:expr }),*) => {

        impl $crate::expr::ops::Op2 for $s {
            fn cost(&self) -> usize { self.0 }
            fn try_eval(&self, a1 : $crate::value::Value, a2 : $crate::value::Value) -> (bool, crate::value::Value) {
                match (a1, a2) { 
                    $(
                        (crate::value::Value::$t1(s1), crate::value::Value::$t2(s2)) => (true, crate::value::Value::$rt(itertools::izip!(s1.iter(), s2.iter()).map($f).galloc_scollect())),
                    )*
                    _ => (false, crate::value::Value::Null),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_op2_opt {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident) -> $rt:ident { $f:expr }),*) => {

        impl $crate::expr::ops::Op2 for $s {
            fn cost(&self) -> usize { self.0 }
            fn try_eval(&self, a1 : $crate::value::Value, a2 : $crate::value::Value) -> (bool, crate::value::Value) {
                match (a1, a2) {
                    $(
                        (crate::value::Value::$t1(s1), crate::value::Value::$t2(s2)) => {
                            let mut flag = true;
                            let a = itertools::izip!(s1.iter(), s2.iter()).map($f).map(|f| { flag &= f.is_some(); f.unwrap_or($crate::default_value![$rt]) }).galloc_scollect();
                            (flag, crate::value::Value::$rt(a))
                        }
                    )*
                    _ => (false, crate::value::Value::Null),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_op3 {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident, $t3:ident) -> $rt:ident { $f:expr }),*) => {

        impl $crate::expr::ops::Op3 for $s {
            fn cost(&self) -> usize { self.0 }
            fn try_eval(&self, a1 : $crate::value::Value, a2 : $crate::value::Value, a3 : crate::value::Value) -> (bool, crate::value::Value) {
                match (a1, a2, a3) {
                    $(
                        (crate::value::Value::$t1(s1), crate::value::Value::$t2(s2), crate::value::Value::$t3(s3)) =>
                            (true, crate::value::Value::$rt(itertools::izip!(s1.iter(), s2.iter(), s3.iter()).map($f).galloc_scollect())),
                    )*
                    _ => (false, crate::value::Value::Null),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_op3_opt {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident, $t3:ident) -> $rt:ident { $f:expr }),*) => {

        impl $crate::expr::ops::Op3 for $s {
            fn cost(&self) -> usize { self.0 }
            fn try_eval(&self, a1 : $crate::value::Value, a2 : $crate::value::Value, a3 : crate::value::Value) -> (bool, crate::value::Value) {
                match (a1, a2, a3) {
                    $(
                        (crate::value::Value::$t1(s1), crate::value::Value::$t2(s2), crate::value::Value::$t3(s3)) => {
                            let mut flag = true
                            let a = itertools::izip!(s1.iter(), s2.iter(), s3.iter()).map($f).map(|f| { flag &= f.is_some(); f.unwrap_or($crate::default_value![$rt]) }).galloc_scollect();
                            (flag, crate::value::Value::$rt(a))
                        }
                    )*
                    _ => (false, crate::value::Value::Null),
                }
            }
        }
    };
}
