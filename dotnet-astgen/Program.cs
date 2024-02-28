namespace Foo {

 class ImplementationClass
 {
     // Explicit interface member implementation:
      static void Main()
      {
        int[] numbers = { 2, 3, 4, 5 };
        var squaredNumbers = numbers.Select(x => x * x);
      }
 }
}