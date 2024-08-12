package com.github.johnmurray.mavir;

import com.google.auto.value.AutoValue;

@AutoValue
public abstract class TestClass {
    abstract String name();

    public abstract long longValue();

    protected abstract int intValue();

    public abstract float floatValue();

    public abstract double doubleValue();

    // Boolean method
    public abstract boolean booleanValue();

    /** Char method */
    public abstract char charValue();

    public TestClass create(String name, long longValue, int intValue, float floatValue, double doubleValue, boolean booleanValue, char charValue) {
        return new AutoValue_TestClass(name, longValue, intValue, floatValue, doubleValue, booleanValue, charValue);
    }
}
