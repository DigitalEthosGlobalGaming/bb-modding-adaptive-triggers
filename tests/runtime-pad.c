#define BB_MODHOST 1
#define SDL_SendGamepadEffect test_send
#define SDL_SetGamepadLED test_led
#define SDL_GamepadConnected test_connected
#define SDL_GetGamepadType test_type
#include "src/runtime_pad.c"
#include <assert.h>
static uint8_t packet[47], rgb[3];
static int sends, leds;
static SDL_GamepadType kind=SDL_GAMEPAD_TYPE_PS5;
bool test_send(SDL_Gamepad *g,const void *data,int size){assert(g==gamepad);assert(size==47);memcpy(packet,data,47);sends++;return true;}
bool test_led(SDL_Gamepad *g,Uint8 r,Uint8 green,Uint8 b){assert(g==gamepad);rgb[0]=r;rgb[1]=green;rgb[2]=b;leds++;return true;}
bool test_connected(SDL_Gamepad *g){(void)g;return true;}
SDL_GamepadType test_type(SDL_Gamepad *g){(void)g;return kind;}
int bbgpu_overlay_captures_input(void){return 0;}
void bbgpu_mouse_take(BbMouseInput*out){memset(out,0,sizeof *out);}
void bbgpu_input_configure(int a,int32_t b,int32_t c){(void)a;(void)b;(void)c;}
void bbgpu_prompts_configure(const char *const *inputs,int count){(void)inputs;(void)count;}
void bbgpu_mouse_camera_configure(int a,float b,float x,float y,int sync){(void)a;(void)b;(void)x;(void)y;(void)sync;}
int bbgpu_input_reload_requested(void){return 0;}
uintptr_t runtime_lookup(const RuntimeExport*t,size_t n,const char*s){(void)t;(void)n;(void)s;return 0;}
void runtime_modhost_pad(uint32_t b){(void)b;}
void runtime_modhost_frame(void){}
int main(void){
 sdl_ready=1;gamepad=(SDL_Gamepad*)(uintptr_t)1;
 uint8_t r[11]={1,0,110},l[11]={6,15,63,128},off[11]={5},expected[47]={0};
 expected[0]=12;memcpy(expected+10,r,11);memcpy(expected+21,l,11);
 assert(runtime_pad_effect_send(NULL,r,l)==0);assert(!memcmp(packet,expected,47));
 assert(runtime_pad_effect_send(NULL,NULL,l)==0);assert(packet[0]==8);
 for(int i=10;i<21;i++)assert(packet[i]==0);
 assert(runtime_pad_effect_led(NULL,120,0,10)==0);assert(leds==1&&rgb[0]==120&&rgb[2]==10);
 assert(runtime_pad_effect_send(NULL,off,off)==0);assert(packet[10]==5&&packet[21]==5);
 kind=SDL_GAMEPAD_TYPE_XBOXONE;int before=sends;
 assert(runtime_pad_effect_send(NULL,r,l)==-1&&sends==before);
 assert(runtime_pad_effect_led(NULL,1,2,3)==-1&&leds==1);
 puts("runtime_pad SDL packet, LED, off and non-DualSense checks: OK");return 0;
}
