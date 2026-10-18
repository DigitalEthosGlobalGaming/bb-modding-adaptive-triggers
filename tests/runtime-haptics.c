#include <SDL3/SDL.h>
#include <assert.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
static int channels=4, outputs=1, pads=1, wired=1, ps5=1, writes, closes, fail_write;
static uint64_t ticks=10000;
static SDL_JoystickID pad_id=7;
static float copied[16];
static SDL_JoystickID *pad_ids(int *n){*n=pads;SDL_JoystickID*p=SDL_malloc(sizeof(*p)*pads);for(int i=0;i<pads;i++)p[i]=7+i;return p;}
static SDL_AudioDeviceID *audio_ids(int *n){*n=outputs;SDL_AudioDeviceID*p=SDL_malloc(sizeof(*p)*outputs);for(int i=0;i<outputs;i++)p[i]=100+i;return p;}
static const char *audio_name(SDL_AudioDeviceID id){assert(id>=100);return "Speakers (2- DualSense Wireless Controller)";}
static bool audio_format(SDL_AudioDeviceID id,SDL_AudioSpec*s,int*n){(void)id;(void)n;*s=(SDL_AudioSpec){SDL_AUDIO_F32,channels,48000};return true;}
static SDL_AudioStream *audio_open(SDL_AudioDeviceID id,const SDL_AudioSpec*s,SDL_AudioStreamCallback cb,void*u){assert(id==100 && s->channels==4 && s->freq==48000 && s->format==SDL_AUDIO_F32 && !cb && !u);return (SDL_AudioStream*)(uintptr_t)1;}
static void audio_close(SDL_AudioStream*s){assert(s);closes++;}
static bool put(SDL_AudioStream*s,const void*p,int n){assert(s && n==sizeof(copied));memcpy(copied,p,n);writes++;return !fail_write;}
static bool rumble(SDL_Gamepad*g,Uint16 l,Uint16 r,Uint32 ms){assert(g && !l && !r && !ms);return true;}
#define SDL_WasInit(x) (x)
#define SDL_GetTicks() ticks
#define SDL_GetGamepads pad_ids
#define SDL_GetGamepadTypeForID(id) SDL_GAMEPAD_TYPE_PS5
#define SDL_GetGamepadID(g) ((void)(g),pad_id)
#define SDL_GetGamepadType(g) (ps5 ? SDL_GAMEPAD_TYPE_PS5 : SDL_GAMEPAD_TYPE_XBOXONE)
#define SDL_GetGamepadConnectionState(g) (wired ? SDL_JOYSTICK_CONNECTION_WIRED : SDL_JOYSTICK_CONNECTION_WIRELESS)
#define SDL_GetAudioPlaybackDevices audio_ids
#define SDL_GetAudioDeviceName audio_name
#define SDL_GetAudioDeviceFormat audio_format
#define SDL_OpenAudioDeviceStream audio_open
#define SDL_GetAudioStreamDevice(s) 100
#define SDL_DestroyAudioStream audio_close
#define SDL_RumbleGamepad rumble
#define SDL_ClearAudioStream(s) true
#define SDL_PutAudioStreamData put
#define SDL_FlushAudioStream(s) true
#define SDL_ResumeAudioStreamDevice(s) true
#include "src/runtime_haptics.inc"
int main(void){
 SDL_Gamepad*g=(SDL_Gamepad*)(uintptr_t)1;float pcm[8]={0.1f,0.2f,-0.1f,-0.2f,0.3f,0.4f,0,0};
 assert(grip_play(g,pcm,4)==0 && writes==1);
 for(int i=0;i<4;i++){assert(copied[i*4]==0 && copied[i*4+1]==0);assert(copied[i*4+2]==pcm[i*2] && copied[i*4+3]==pcm[i*2+1]);}
 assert(grip_until_ms>ticks && grip_until_ms<=ticks+31);
 assert(grip_play(NULL,NULL,0)==0 && !grip_stream && !grip_until_ms && closes>0);
 outputs=2;ticks+=4000;assert(grip_play(g,pcm,4)==-1 && !grip_stream);outputs=1;
 pads=2;ticks+=4000;assert(grip_play(g,pcm,4)==-1 && !grip_stream);pads=1;
 channels=2;ticks+=4000;assert(grip_play(g,pcm,4)==-1 && !grip_stream);channels=4;
 wired=0;ticks+=4000;assert(grip_play(g,pcm,4)==-1);wired=1;
 ps5=0;assert(grip_play(g,pcm,4)==-1);ps5=1;
 assert(grip_play(g,pcm,4)==0);channels=2;
 assert(grip_play(g,pcm,4)==-1 && !grip_stream);channels=4;
 ticks+=4000;fail_write=1;assert(grip_play(g,pcm,4)==-1 && !grip_stream && !grip_until_ms);
 puts("USB grip audio: quad routing, no default output, ambiguity, format changes, failure and cleanup: OK");
}
