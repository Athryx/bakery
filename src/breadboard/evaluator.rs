use core::fmt::{self, Display, Write};

use uuid::{Uuid, uuid};

use crate::ftd_data::{SectionData, DataEntry};
use super::{BNumber, Breadboard, Component, Line, LineInner, LineValue};

#[derive(Debug, Default)]
pub struct Evaluator {
    inputs: Vec<LineInner>,
    // there is one expression for each output
    exprs: Vec<EvaluatorExpression>,
}

impl Evaluator {
    /// Returns an expression representing the given input
    pub fn get_input(&mut self, input_line: LineInner) -> Option<EvaluatorExpression> {
        for (i, input) in self.inputs.iter().enumerate() {
            if input_line == *input {
                return EvaluatorExpression::from_input_index(i);
            }
        }

        let expr = EvaluatorExpression::from_input_index(self.inputs.len())?;
        self.inputs.push(input_line);
        Some(expr)
    }
}

impl Component for Evaluator {
    fn ftd_uuid(&self) -> Uuid {
        uuid!("7cf3b706-757e-428a-bb45-454a17ed710a")
    }

    fn section_data(&self) -> SectionData {
        let mut expr_string = String::new();
        for (i, expr) in self.exprs.iter().enumerate() {
            if i != 0 {
                expr_string.push_str(",");
            }

            write!(expr_string, "{expr}").unwrap();
        }

        SectionData::default()
            .with_entry(0, DataEntry::String(expr_string))
    }

    fn num_outputs(&self) -> usize {
        self.exprs.len()
    }

    fn inputs(&self) -> &[LineInner] {
        self.inputs.as_slice()
    }
}

macro_rules! make_bb_method {
    ($name:ident, $variant:ident, $in_type:ty, $out_type:ty) => {
        fn $name<'a>(&'a self, input: Line<'a, $in_type>) -> Line<'a, $out_type> {
            let mut eval = Evaluator::default();

            let input_expr = eval.get_input(input.inner).unwrap();
            eval.exprs.push(EvaluatorExpression::$variant(Box::new(input_expr)));

            self.insert_component_with_output(eval)
        }
    };

    ($name:ident, $variant:ident, $in_type1:ty, $in_type2:ty, $out_type:ty) => {
        fn $name<'a>(&'a self, input1: Line<'a, $in_type1>, input2: Line<'a, $in_type2>) -> Line<'a, $out_type> {
            let mut eval = Evaluator::default();

            let input_expr1 = eval.get_input(input1.inner).unwrap();
            let input_expr2 = eval.get_input(input2.inner).unwrap();
            eval.exprs.push(EvaluatorExpression::$variant(Box::new(input_expr1), Box::new(input_expr2)));

            self.insert_component_with_output(eval)
        }
    };
}

impl Breadboard {
    make_bb_method!(sin, Sin, BNumber, BNumber);
    make_bb_method!(cos, Cos, BNumber, BNumber);
    make_bb_method!(tan, Tan, BNumber, BNumber);
    make_bb_method!(sqrt, Sqrt, BNumber, BNumber);
    make_bb_method!(asin, Asin, BNumber, BNumber);
    make_bb_method!(acos, Acos, BNumber, BNumber);
    make_bb_method!(atan, Atan, BNumber, BNumber);
}

#[derive(Debug)]
pub enum EvaluatorExpression {
    InputA,
    InputB,
    InputC,
    InputD,
    InputE,
    Sin(Box<Self>),
    Cos(Box<Self>),
    Tan(Box<Self>),
    Sqrt(Box<Self>),
    Asin(Box<Self>),
    Acos(Box<Self>),
    Atan(Box<Self>),
    Atan2(Box<Self>, Box<Self>),
    Exp(Box<Self>),
    Log(Box<Self>),
    Pow(Box<Self>, Box<Self>),
    // absolute value also works component wise on a vector
    Abs(Box<Self>),
    Sign(Box<Self>),
    Round(Box<Self>),
    Floor(Box<Self>),
    Ceil(Box<Self>),
    Max2(Box<Self>, Box<Self>),
    Max3(Box<Self>, Box<Self>, Box<Self>),
    // returns maximum component of the vector
    MaxV(Box<Self>),
    Min2(Box<Self>, Box<Self>),
    Min3(Box<Self>, Box<Self>, Box<Self>),
    // returns minimum component of the vector
    MinV(Box<Self>),
    If {
        condition: Box<Self>,
        true_value: Box<Self>,
        false_value: Box<Self>,
    },
    Vector(Box<Self>, Box<Self>, Box<Self>),
    MakeRotationBetween {
        from_vector: Box<Self>,
        to_vector: Box<Self>,
    },
    FromEuler {
        pitch: Box<Self>,
        yaw: Box<Self>,
        roll: Box<Self>,
    },
    // crates rotation with x being x rotation, y rotation, and z rotation
    FromEularV(Box<Self>),
    // converts rotation back into vector of angles
    ToEularV(Box<Self>),
    // returns angle of rotation (based on real part of quaternion?)
    Angle(Box<Self>),
    // returns the rotation axis of quaternion
    Axis(Box<Self>),
    AngleBetween {
        from_vector: Box<Self>,
        to_vector: Box<Self>,
    },
    SetX {
        vector: Box<Self>,
        x: Box<Self>,
    },
    SetY {
        vector: Box<Self>,
        y: Box<Self>,
    },
    SetZ {
        vector: Box<Self>,
        z: Box<Self>,
    },
    // output vector of previous frame for the given index into the outputs
    OutputV(Box<Self>),
    // output number of previous frame for the given index into the outputs
    Output(Box<Self>),
    GetX(Box<Self>),
    GetY(Box<Self>),
    GetZ(Box<Self>),
    Magnitude(Box<Self>),
    SqruareMagnitude(Box<Self>),
    RotationInverse(Box<Self>),
    // works for 2 numbers, string, or vectors
    Add(Box<Self>, Box<Self>),
    // works for 2 numbers, strings (lhs with instances of rhs removed), or vectors
    Sub(Box<Self>,Box<Self>),
    // vector cross product
    Cross(Box<Self>, Box<Self>),
    // works for 2 numbers, 2 vectors (dot product), rotation then vector (rotate vector),
    // number then vector, vector then number, 2 rotations (new rotation where lhs rotate after rhs)
    Mul(Box<Self>, Box<Self>),
    // works for 2 numbers, vector / number, vector / rotation (apply inverse rotation)
    Div(Box<Self>, Box<Self>),
    // workse for 2 numbers (remainder)
    Mod(Box<Self>, Box<Self>),
    Eq(Box<Self>, Box<Self>),
    Ne(Box<Self>, Box<Self>),
    Gt(Box<Self>, Box<Self>),
    Gte(Box<Self>, Box<Self>),
    Lt(Box<Self>, Box<Self>),
    Lte(Box<Self>, Box<Self>),
    OpAnd(Box<Self>, Box<Self>),
    OpOr(Box<Self>, Box<Self>),
    FalseCoalesce(Box<Self>, Box<Self>),
    Negate(Box<Self>),
}

impl EvaluatorExpression {
    fn from_input_index(n: usize) -> Option<Self> {
        match n {
            0 => Some(Self::InputA),
            1 => Some(Self::InputB),
            2 => Some(Self::InputC),
            3 => Some(Self::InputD),
            4 => Some(Self::InputE),
            _ => None,
        }
    }
}

impl Display for EvaluatorExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputA => write!(f, "a"),
            Self::InputB => write!(f, "b"),
            Self::InputC => write!(f, "c"),
            Self::InputD => write!(f, "d"),
            Self::InputE => write!(f, "e"),
            Self::Sin(val) => write!(f, "Sin({val})"),
            Self::Cos(val) => write!(f, "Cos({val})"),
            Self::Tan(val) => write!(f, "Tan({val})"),
            Self::Sqrt(val) => write!(f, "Sqrt({val})"),
            Self::Asin(val) => write!(f, "Asin({val})"),
            Self::Acos(val) => write!(f, "Acos({val})"),
            Self::Atan(val) => write!(f, "Atan({val})"),
            Self::Atan2(val1, val2) => write!(f, "Atan({val1}, {val2})"),
            Self::Exp(val) => write!(f, "Exp({val})"),
            Self::Log(val) => write!(f, "Log({val})"),
            Self::Pow(val1, val2) => write!(f, "Pow({val1}, {val2})"),
            Self::Abs(val) => write!(f, "Abs({val})"),
            Self::Sign(val) => write!(f, "Sign({val})"),
            Self::Round(val) => write!(f, "Round({val})"),
            Self::Floor(val) => write!(f, "Floor({val})"),
            Self::Ceil(val) => write!(f, "Ceil({val})"),
            Self::Max2(val1, val2) => write!(f, "Max({val1}, {val2})"),
            Self::Max3(val1, val2, val3) => write!(f, "Max({val1}, {val2}, {val3})"),
            Self::MaxV(val) => write!(f, "Max({val})"),
            Self::Min2(val1, val2) => write!(f, "Min({val1}, {val2})"),
            Self::Min3(val1, val2, val3) => write!(f, "Min({val1}, {val2}, {val3})"),
            Self::MinV(val) => write!(f, "Min({val})"),
            Self::If {
                condition,
                true_value,
                false_value,
            } => write!(f, "If({condition}, {true_value}, {false_value})"),
            Self::Vector(val1, val2, val3) => write!(f, "Vector({val1}, {val2}, {val3})"),
            Self::MakeRotationBetween {
                from_vector,
                to_vector,
            } => write!(f, "FromToRot({from_vector}, {to_vector}"),
            Self::FromEuler {
                pitch,
                yaw,
                roll,
            } => write!(f, "FromEuler({pitch}, {yaw}, {roll})"),
            Self::FromEularV(val) => write!(f, "FromEuler({val})"),
            Self::ToEularV(val) => write!(f, "ToEuler({val})"),
            Self::Angle(val) => write!(f, "Angle({val})"),
            Self::Axis(val) => write!(f, "Axis({val})"),
            Self::AngleBetween {
                from_vector,
                to_vector,
            } => write!(f, "Angle({from_vector}, {to_vector})"),
            Self::SetX {
                vector,
                x,
            } => write!(f, "setX({vector}, {x})"),
            Self::SetY {
                vector,
                y,
            } => write!(f, "setX({vector}, {y})"),
            Self::SetZ {
                vector,
                z,
            } => write!(f, "setX({vector}, {z})"),
            Self::OutputV(val) => write!(f, "outputV({val})"),
            Self::Output(val) => write!(f, "output({val})"),
            Self::GetX(val) => write!(f, "({val}).x"),
            Self::GetY(val) => write!(f, "({val}).y"),
            Self::GetZ(val) => write!(f, "({val}).z"),
            Self::Magnitude(val) => write!(f, "({val}).magnitude"),
            Self::SqruareMagnitude(val) => write!(f, "({val}).sqrMagnitude"),
            Self::RotationInverse(val) => write!(f, "({val}).inverse"),
            Self::Add(lhs, rhs) => write!(f, "({lhs}) + ({rhs})"),
            Self::Sub(lhs, rhs) => write!(f, "({lhs}) - ({rhs})"),
            Self::Cross(lhs, rhs) => write!(f, "({lhs}) x ({rhs})"),
            Self::Mul(lhs, rhs) => write!(f, "({lhs}) * ({rhs})"),
            Self::Div(lhs, rhs) => write!(f, "({lhs}) / ({rhs})"),
            Self::Mod(lhs, rhs) => write!(f, "({lhs}) % ({rhs})"),
            Self::Eq(lhs, rhs) => write!(f, "({lhs}) = ({rhs})"),
            Self::Ne(lhs, rhs) => write!(f, "({lhs}) != ({rhs})"),
            Self::Gt(lhs, rhs) => write!(f, "({lhs}) > ({rhs})"),
            Self::Gte(lhs, rhs) => write!(f, "({lhs}) >= ({rhs})"),
            Self::Lt(lhs, rhs) => write!(f, "({lhs}) < ({rhs})"),
            Self::Lte(lhs, rhs) => write!(f, "({lhs}) <= ({rhs})"),
            Self::OpAnd(lhs, rhs) => write!(f, "({lhs}) & ({rhs})"),
            Self::OpOr(lhs, rhs) => write!(f, "({lhs}) | ({rhs})"),
            Self::FalseCoalesce(lhs, rhs) => write!(f, "({lhs}) or ({rhs})"),
            Self::Negate(val) => write!(f, "-({val})"),
        }
    }
}