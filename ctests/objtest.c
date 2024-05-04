#include "./ioworking.h"

void string_destruct(object* str) {
   free(str->content->string);
   free(str->content);
   free(str);
}

void rc_decrement(struct object* rc) {
   rc->refcount--;
   if (rc->refcount==0) {
      rc->destruct(rc);
   }
}

void rc_increment(object* rc) {
   rc->refcount++;
}

string new_string() {
   Data* newstr = (Data*) malloc(sizeof(Data));
   object* strobj = (object*) malloc(sizeof(object));
   newstr->string = (char*) malloc(sizeof(char)*16);
   strobj->content = newstr;
   strobj->type = STRING;
   strobj->destruct = &string_destruct;
   strobj->refcount = 0;

   rc_increment(strobj);
   return strobj;
}

void push_string(string dest, char src[]) {
   // Because the function returns the reference counted pointer,
   // it calls rc_increment just before returning it. This means
   // that you must not call rc_increment when receiving it.
   size_t srclen = 0;
   while (src[srclen] != '\0') {
      srclen++;
   }
   size_t targlen = 0;
   while (dest->content->c[targlen] != '\0') {
      targlen++;
   }
   dest->content->c = realloc(dest->content->c, sizeof(char)*srclen+targlen+1);
   for (size_t i = 0; i < srclen; i++) {
      dest->content->c[i+targlen] = src[i];
   }
   dest->content->c[srclen+targlen] = '\0';
}

char* input() {
   int ch;
   object* str = new_string();
   while ((ch = getchar()) != EOF && ch != '\n') {
      char* temparr = malloc(sizeof(char)*2);
      temparr[0] = ch;
      temparr[1] = '\0';
      push_string(str, temparr);
      free(temparr);
   }
   rc_increment(str);
   rc_decrement(str);
   return str->content->c;
}

int main() {
   struct object* mystring = new_string();
   char* src = "hello";
   push_string(mystring, src);
   src = " world";
   push_string(mystring, src);
   printf("%s\n", mystring->content->c);
   rc_decrement(mystring);
}
