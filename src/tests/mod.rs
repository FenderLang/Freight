use crate::{
    execution_engine::ExecutionEngine,
    expression::Expression,
    function::{ArgCount, FunctionWriter},
};

use self::type_system::{TestBinaryOperator, TestTypeSystem, TestValue, TestValueWrapper};

mod type_system;

#[test]
fn test_functions() {
    let mut engine = ExecutionEngine::<TestTypeSystem>::new_default();
    let mut add = FunctionWriter::new(ArgCount::Fixed(2));
    let a = 0;
    let b = 1;
    add.evaluate_expression(Expression::BinaryOpEval(
        TestBinaryOperator::Add,
        [Expression::stack(a), Expression::stack(b)].into(),
    ));
    let add = engine.register_function(add, 0);
    let mut main = FunctionWriter::new(ArgCount::Fixed(0));
    let x = main.create_variable();
    let y = main.create_variable();
    main.evaluate_expression(Expression::AssignStack(
        x,
        Expression::RawValue(TestValueWrapper(TestValue::Number(3))).into(),
    ));
    main.evaluate_expression(Expression::AssignStack(
        y,
        Expression::RawValue(TestValueWrapper(TestValue::Number(2))).into(),
    ));
    main.evaluate_expression(Expression::StaticFunctionCall(
        add,
        vec![Expression::stack(x), Expression::stack(y)],
    ));
    let main = engine.register_function(main, 0);
    assert_eq!(
        engine.call(&main, []).unwrap(),
        TestValueWrapper(TestValue::Number(5))
    );
}
