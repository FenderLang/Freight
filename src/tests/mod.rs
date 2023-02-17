use crate::{
    execution_context::register_ids::RegisterId,
    expression::{Expression, Operand},
    function::FunctionBuilder,
    vm_writer::VMWriter,
};

use self::type_system::{TestBinaryOperator, TestTypeSystem, TestValue, TestValueWrapper};

mod type_system;

#[test]
fn test_functions() {
    let mut writer = VMWriter::<TestTypeSystem>::new();
    let mut add = FunctionBuilder::new(2);
    let a = add.argument_stack_offset(0);
    let b = add.argument_stack_offset(1);
    add.return_expression(Expression::BinaryOpEval {
        operator: TestBinaryOperator::Add,
        right_operand: Operand::ValueRef(a),
        left_operand: Operand::ValueRef(b),
    })
    .unwrap();
    let add = writer.include_function(add);
    let mut main = FunctionBuilder::new(0);
    let x = main.create_variable();
    let y = main.create_variable();
    main.assign_value(
        x,
        Expression::Eval(Operand::ValueRaw(TestValueWrapper(TestValue::Number(3)))),
    )
    .unwrap();
    main.assign_value(
        y,
        Expression::Eval(Operand::ValueRaw(TestValueWrapper(TestValue::Number(2)))),
    )
    .unwrap();
    main.return_expression(Expression::Eval(Operand::StaticFunctionCall {
        function: add,
        args: vec![Operand::ValueRef(x), Operand::ValueRef(y)],
    }))
    .unwrap();
    let main = writer.include_function(main);
    let mut vm = writer.finish(main);
    vm.run().unwrap();
    assert_eq!(
        vm.get_register(RegisterId::Return),
        &TestValueWrapper(TestValue::Number(5))
    );
    assert_eq!(vm.stack_size(), 3);
}