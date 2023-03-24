use crate::{expression::Expression, function::{FunctionWriter, ArgCount}, vm_writer::VMWriter};

use self::type_system::{TestBinaryOperator, TestTypeSystem, TestValue, TestValueWrapper};

mod type_system;

#[test]
fn test_functions() {
    let mut writer = VMWriter::<TestTypeSystem>::new();
    let mut add = FunctionWriter::new(ArgCount::Fixed(2));
    let a = 0;
    let b = 1;
    add.evaluate_expression(Expression::BinaryOpEval(
        TestBinaryOperator::Add,
        [Expression::stack(a), Expression::stack(b)].into(),
    ));
    let add = writer.include_function(add, 0);
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
    let main = writer.include_function(main, 0);
    let mut vm = writer.finish_default(main);
    assert_eq!(vm.run().unwrap(), TestValueWrapper(TestValue::Number(5)));
}
