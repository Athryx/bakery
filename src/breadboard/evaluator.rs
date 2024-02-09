use core::fmt::{self, Display, Write};

use uuid::{Uuid, uuid};

use crate::ftd_data::{SectionData, DataEntry};
use super::{Component, LineInner};

#[derive(Debug)]
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
        }
    }
}