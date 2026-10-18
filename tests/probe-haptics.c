/* Read-only endpoint probe; deliberately never submits a waveform. */
#include <SDL3/SDL.h>
#include <stdio.h>
int main(void) {
 if(!SDL_Init(SDL_INIT_GAMEPAD|SDL_INIT_AUDIO)){fprintf(stderr,"%s\n",SDL_GetError());return 1;}
 SDL_PumpEvents();int n=0;SDL_JoystickID*ids=SDL_GetGamepads(&n);
 for(int i=0;ids&&i<n;i++){SDL_Gamepad*g=SDL_OpenGamepad(ids[i]);if(g){printf("Pad: %s type=%d connection=%d\n",SDL_GetGamepadName(g),SDL_GetGamepadType(g),SDL_GetGamepadConnectionState(g));SDL_CloseGamepad(g);}}
 SDL_free(ids);SDL_AudioDeviceID*audio=SDL_GetAudioPlaybackDevices(&n);
 for(int i=0;audio&&i<n;i++){SDL_AudioSpec f;if(SDL_GetAudioDeviceFormat(audio[i],&f,NULL))printf("Audio: %s channels=%d rate=%d\n",SDL_GetAudioDeviceName(audio[i]),f.channels,f.freq);}
 SDL_free(audio);SDL_Quit();return 0;
}
