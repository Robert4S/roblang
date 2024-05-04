#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>

/**
 * @brief This function prints a formatted string with integer placeholders replaced by provided integers.
 *
 * The function takes a format string and a variable number of integer arguments. It replaces each "{}" placeholder
 * in the format string with the next provided integer. The list of integers must be terminated with INT_MAX.
 * showme_text("One is {}, Two is {}, Three is {}", 1, 2, 3, INT_MAX) -> "One is 1, Two is 2, Three is 3"
 * @param format The format string, containing "{}" placeholders for integers.
 * @param ... The integers to replace the placeholders. The list must be terminated with INT_MAX.
 */
void showme_int(const char* format, ...);

/**
 * @brief This function prints a formatted string with text placeholders replaced by provided strings.
 *
 * The function takes a format string and a variable number of string arguments. It replaces each "{}" placeholder
 * in the format string with the next provided string. The list of strings must be terminated with NULL.
 * showme_text("{} {}{}", "hello", "world", "!", NULL) -> "hello world!"
 * @param format The format string, containing "{}" placeholders for strings.
 * @param ... The strings to replace the placeholders. The list must be terminated with NULL.
 */
void showme_text(const char* format, ...);

typedef struct object object;

typedef union {
    int i_num;
    char* string;
} Data;

enum Type {
	INT,
	STRING,
};

struct object {
    Data* content;
    enum Type type;
    unsigned long refcount;
    void (*destruct)(object*);
};

typedef object* string;
typedef object* num;



inline void showme_int(const char* format, ...) {
    va_list args;
    va_start(args, format);

    int value;
    while ((value = va_arg(args, int)) != INT_MAX) {
	const char* placeholder = strstr(format, "{}");
	if (placeholder != NULL) {
	    printf("%.*s%d", (int)(placeholder - format), format, value);
	    format = placeholder + 2; // Skip past the placeholder
	}
    }

    printf("%s", format); // Print the rest of the format string

    va_end(args);
}

inline void showme_text(const char* format, ...) {
    va_list args;
    va_start(args, format);

    char* value;
    while ((value = va_arg(args, char*)) != NULL ) {
	const char* placeholder = strstr(format, "{}");
	if (placeholder != NULL) {
	    printf("%.*s%s", (int)(placeholder - format), format, value);
	    format = placeholder + 2;
	}
    }
    printf("%s", format);

    va_end(args);
}


inline void showme(const char* format, object* myobj) {
    switch (myobj->type) {
	case STRING:
	    showme_text(format, myobj->content->string, NULL);
	break;
	case INT:
	    showme_int(format, myobj->content->i_num, INT_MAX);
	break;
    }
}
