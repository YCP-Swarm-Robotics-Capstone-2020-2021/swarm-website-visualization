//! Various macros to redeclare a variable with a new value
//! Used to reduce boilerplate code of `let x = x.clone(); let y = y.clone();`, etc.

macro_rules! clone
{
    ($($var:ident),+) =>
    {
        $(
            let $var = $var.clone();
        )+
    };
    ($($parent:ident.$var:ident),+) =>
    {
        $(
            let $var = $parent.$var.clone();
        )+
    };
}

macro_rules! wrap
{
    ($($var:ident),+) =>
    {
        $(
            let $var = Rc::new(RefCell::new($var));
        )+
    };
    ($($parent:ident.$var:ident),+) =>
    {
        $(
            let $var = Rc::new(RefCell:new($parent.$var));
        )+
    }
}

#[allow(unused_macros)]
macro_rules! borrow
{
    ($($var:ident),+) =>
    {
        $(
            let $var = $var.borrow();
        )+
    };
    ($($parent:ident.$var:ident),+) =>
    {
        $(
            let $var = $parent.$var.borrow();
        )+
    };
}

macro_rules! borrow_mut
{
    ($($var:ident),+) =>
    {
        $(
            let mut $var = $var.borrow_mut();
        )+
    };
    ($($parent:ident.$var:ident),+) =>
    {
        $(
            let $var = $parent.$var.borrow_mut();
        )+
    };
}