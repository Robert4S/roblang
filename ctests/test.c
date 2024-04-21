#include "robIO.h"


int main() {
	int mynum = 5;
	char mychar[] = "Hello world!";

	struct object obj;
	obj.content.c = mychar;
	obj.type = CHAR;
	showme("My object is {}\n", obj);
	obj.content.i = mynum;
	obj.type = INT;
	int* int_vec = vector_create();
	for (int i=0; i<100; i++) {
		if (i%2 == 0) {
			vector_add(&int_vec, i);
		}
	}
	for (int i=0; i<vector_size(int_vec); i++) {
		showme_int("Item no.{} in the vec is {}\n", i, int_vec[i], INT_MAX);
	}
	showme("My object again is {}\n", obj);

	
	showme_text("{}", mychar, NULL);

	return 0;
}
