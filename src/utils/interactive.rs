use anyhow::Result;

pub trait FromInteractive: Sized {
    fn from_interactive() -> Result<Self>;
}

#[macro_export]
macro_rules! get_answer {
    ($answers: expr, $token: expr$(,)?) => {{
        get_answer!($answers, as_string, $token)
    }};
    ($answers: expr, $as_type: ident, $token: expr$(,)?) => {{
        let ans: &Answers = &$answers;
        let token: &str = $token;
        ans.get(token).unwrap().$as_type().unwrap().to_owned()
    }};
}
