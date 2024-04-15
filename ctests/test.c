#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void fizzbuzz() {
	for (int i = 1; i < 20; i++) {
		if (i%5==0 && i%3==0) {
			printf("%d Fizzbuzz\n", i);
		} else if (i%5==0) {
			printf("%d Fizz\n", i);
		} else if (i%3==0) {
			printf("%d Buzz\n", i);
		}
	}
}

int main() {
	fizzbuzz();
	return 0;
}


