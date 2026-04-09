# Phenotype for the generation of Java classes

Code will be generated in the `java/generation` namespace.

```pht
java/generation:
```

## Enumeration of Java visibility specifiers

```pht
type Visibility: [public, protected, private];
```

## Data type to represent a typed argument

```pht
Argument plural Arguments:
    typeName: required string,
    name: required string,
    @(typeName), " ", @(name)
;
```

## Data type to represent a method signature

```pht
MethodSignature plural MethodSignatures:
    methodName: required string,
    visibility: required Visibility,
    returnType: required string,
    argList: optional string,
    @(visibility), " ", @(returnType), " ", @(methodName),
    "(", @(argList)?, ")"
;
```

## Template for Java classes

A `JavaClass` contains an optional superclass, optional methods, and
a nested `Constructor` type that can reference the parent class name.

```pht
JavaClass plural JavaClasses:
    name: required string,
    visibility: required Visibility,
    superClass: optional string,

    Constructor plural Constructors:
        argList: optional string,
        @(JavaClass/name), "(", @(argList)?, ")"
    ;,

    ctors: required Constructors,
    methods: optional MethodSignatures,

    @(visibility), " class ", @(name),
    @ifset(superClass) { " extends ", @(superClass) },
    " {", @eol,
    @join(ctors, "\n"),
    @ifnotempty(methods) { @eol, @join(methods, "\n") },
    @eol, "}"
;
```
