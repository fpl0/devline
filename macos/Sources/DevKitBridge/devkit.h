#ifndef DEVKIT_BRIDGE_H
#define DEVKIT_BRIDGE_H

char *devkit_request(const char *json);
void devkit_free(char *response);

#endif
