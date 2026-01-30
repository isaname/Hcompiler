#[macro_export]
macro_rules! ptr {
    ($tye:ty) => {
        Rc<RefCell<$tye>>
    };
}

#[macro_export]
macro_rules! opt_ptr {
    ($tye:ty) => {
        Option<Rc<RefCell<$tye>>>
    };
}

#[macro_export]
macro_rules! weak_ptr {
    ($tye:ty) => {
        Weak<RefCell<$tye>>
    };
}

#[macro_export]
macro_rules! make_ptr {
    ($name:ident) => {
        Rc::new(RefCell::new($name))
    };
}

#[macro_export]
macro_rules! downgrade {
    ($name:expr) => {
        Rc::downgrade($name)
    };
}

#[macro_export]
macro_rules! create_inst {
    ($name:ident, $id:path, $call:ident) => {
        pub fn $name(v1: ptr!(Value), v2: ptr!(Value), bb: ptr!(Value)) -> Self {
            Self::$call($id, v1, v2, bb)
        }
    };
}

#[macro_export]
macro_rules! module_ptr {
    ($basicblock:ident) => {
        ModulePtr($basicblock.borrow().bb_get_module().unwrap())
    };
}