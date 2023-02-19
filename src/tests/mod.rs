use crate::{expression::Expression, function::FunctionWriter, vm_writer::VMWriter};

use self::type_system::{TestBinaryOperator, TestTypeSystem, TestValue, TestValueWrapper};

mod type_system;

#[test]
fn test_functions() {
    let mut writer = VMWriter::<TestTypeSystem>::new();
    let mut add = FunctionWriter::new(2);
    let a = add.argument_stack_offset(0);
    let b = add.argument_stack_offset(1);
    add.return_expression(Expression::BinaryOpEval(
        TestBinaryOperator::Add,
        Expression::Variable(a).into(),
        Expression::Variable(b).into(),
    ))
    .unwrap();
    let add = writer.include_function(add);
    let mut main = FunctionWriter::new(0);
    let x = main.create_variable();
    let y = main.create_variable();
    main.assign_value(
        x,
        Expression::RawValue(TestValueWrapper(TestValue::Number(3))),
    )
    .unwrap();
    main.assign_value(
        y,
        Expression::RawValue(TestValueWrapper(TestValue::Number(2))),
    )
    .unwrap();
    main.return_expression(Expression::StaticFunctionCall(
        add,
        vec![Expression::Variable(x), Expression::Variable(y)],
    ))
    .unwrap();
    let main = writer.include_function(main);
    let mut vm = writer.finish(main);
    assert_eq!(vm.run().unwrap(), &TestValueWrapper(TestValue::Number(5)));
    assert_eq!(vm.stack_size(), 2);
}
