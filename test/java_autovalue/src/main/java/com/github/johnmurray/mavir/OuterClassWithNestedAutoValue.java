package com.github.johnmurray.mavir;

import com.google.auto.value.AutoValue;

/**
 * Validate several properties in this test:
 *   - Nested classes are generated as expected
 *   - There can be multipel nested classes
 *   - The order of nested classes does not matter with mixed with
 *     other class elements (methods, properties, etc)
 */
class OuterClassWithNestedAutoValue {

    @AutoValue
    static public abstract class OtherNestedTestClass {
        abstract String key();
        abstract Integer value();
    }

    private final String value = "outer-class-value";

    private void doSomething() {
        /* method to ensure that the AutoValue class can be nested anywhere in the class */
        java.lang.System.out.println("doSomething()");
    }

    @AutoValue
    static protected abstract class NestedTestClass {
        abstract String name();

        public abstract long longValue();

        protected abstract int intValue();

        public abstract float floatValue();

        public abstract double doubleValue();

        // Boolean method
        public abstract boolean booleanValue();

        /** Char method */
        public abstract char charValue();

        public NestedTestClass create(String name, long longValue, int intValue, float floatValue, double doubleValue, boolean booleanValue, char charValue) {
            return new AutoValue_OuterClassWithNestedAutoValue_NestedTestClass(
                    name, longValue, intValue, floatValue, doubleValue, booleanValue, charValue);
        }
    }

    public boolean returnSomething() {
        /* method to ensure that the AutoValue class can be nested anywhere in the class */
        return true;
    }
}
