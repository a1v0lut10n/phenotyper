# Phenotype for the generation of java classes

Code will be generated in the java/generation namespace.

```pht
java/generation:
```

## Enumeration of Java visibility specifiers

```pht
Visibility:
    [public, protected, private];
```

## Data type to represent a typed argument

```pht
Argument plural Arguments:
    type: required string,
    name: required string;
```

## Data type to represent a method signature

```pht
MethodSignature plural MethodSignatures:
    methodName: key string,
    visibility: required Visibility,
    returnType: required string,
    argument plural arguments: Arguments;
```

## Template for Java Interfaces

~~~~pht
Interface plural Interfaces:
    name: key string,
    superInterface plural superInterfaces: Interfaces,
    methodSignature plural methodSignatures: MethodSignatures,

    @(1,100):
        @"public interface", @(name),
        ifnotempty(superInterfaces):
            @" extends ",
            for_each(superInterface):
                @(superInterface/name),
                ifmore(superInterfaces):
                    @", "
                ;
            ;
        ;
        @space, @"{", @eol,

        foreach(methodSignature):
            @(+1,100):
                @(methodSignature/visibility), @space, 
                @(methodSignature/returnType), @space,
                @(methodSignature/methodName),
                @"(",
                foreach(methodSignature/argument):
                    methodSignature/argument/type, @space,
                    methodSignature/argument/name,
                    ifmore(methodSignature/arguments):
                        @", "
                    ;
                ;
                @");", @eol
            ;
        @"}", @eol
    ;
;
~~~~

## Template for java classes

~~~~pht
JavaClass plural JavaClasses:
    name: key string,
    interface plural interfaces: optional Interfaces,
    superClass: optional JavaClass,
    methodSignature plural methodSignatures: optional MethodSignatures,

    @(1,100):
        @"public class ", @(name), @space(1),
        ifnotempty(interfaces):
            @"implements ",
            foreach(interface):
                @(interface/name),
                ifmore(interfaces):
                    @", "
                ;
            ;
        ;
        @eol,
        ifset(superClass):
            @(+1,100):
                @" extends ", @(superClass/name), @space
            ;
        ;

        ## Embedded template for Java class constructors.
        Constructor plural Constructors:
            visibility: required Visibility,
            argument plural arguments: optional Arguments,

            @(+1,100):
                @(visibility), @space, @(JavaClass/name), @"(",
                foreach(argument):
                    @(argument/type), @space, @(argument/name),
                    ifmore(arguments):
                        @", "
                    ;
                ;
                @") {", @eol

                @"}", @eol
            ;
        ;

        ## Methods
        methodSignatures?
            foreach(methodSignature):
                @(methodSignature/visibility) @(methodSignature/name)
                // TODO hvwesenbeeck@aivolution.com: complete

    ;
.
~~~~
