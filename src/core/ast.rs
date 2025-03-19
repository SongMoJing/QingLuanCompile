
#[derive(Debug)]
pub enum Statement {
	Expression(Expr),
	Assignment {
		name: String,
		expr: Expr,
	},
}

#[derive(Debug)]
pub enum Expr {
	Number(f64),
	Variable(String),
	Binary {
		left: Box<Expr>,
		op: BinOp,
		right: Box<Expr>,
	},
}

#[derive(Debug)]
pub enum BinOp {
	Add,
	Sub,
	Mul,
	Div,
}