program AddTwo;

var
  a,b,s: integer;

begin
  WriteLn('Enter first integer:');
  ReadLn(a);
  WriteLn('Enter second integer:');
  ReadLn(b);
  s := a + b;
  WriteLn('The sum of a+b is: ', s);
  ReadLn;
end.

