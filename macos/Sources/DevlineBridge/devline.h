#ifndef DEVLINE_BRIDGE_H
#define DEVLINE_BRIDGE_H

char *devline_request(const char *json);
void devline_free(char *response);

#endif
