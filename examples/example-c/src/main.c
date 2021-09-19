//#include <counter/person/sub_counter/Counter.h>
//#include <counter/ffi/rstring/RString.h>
//#include <counter/person/Person.h>
//
//#include <stdio.h>
//#include <string.h>
//
//#define string_assert_eq(a, b) { printf("assert(\"%s\" == \"%s\")\n", a, b); assert(!strcmp(a, b)); }
//
//int main(int argc, char **argv) {
//    Counter counter = Counter_new(2);
//    assert_eq(Counter_get_count(counter), 2);
//    Counter_count(counter, 1);
//    assert_eq(Counter_get_count(counter), 3);
//    Counter_count(counter, 3);
//    assert_eq(Counter_get_count(counter), 6);
//    Counter_drop(counter);
//
//    RString string = RString_new("Hello!");
//    string_assert_eq("Hello!", RString_as_ptr(string));
//    RString_drop(string);
//
//    Person person = Person_new("Danilo", "Guanabara");
//
//    RString fullName = Person_full_name(person);
//    string_assert_eq("Danilo Guanabara", RString_as_ptr(fullName));
//    RString_drop(fullName);
//
//    Person_drop(person);
//
//    return 0;
//}

#include <stdio.h>
#include <example.h>
#include <assert.h>

int main(int argc, char **argv) {
    Instant* instant = now();
    Duration* duration = elapsed(instant);
    print_duration(duration);
    assert(4 == add(1, 3));
    return 0;
}