#include "internal.h"
#include <assert.h>
#include <stdio.h>
typedef struct {int capture;const char *click;int sends;uint8_t bytes[2][11];} Fake;
static int info(void*u,bbm_pad_info*p){(void)u;memset(p,0,sizeof *p);p->device_id=1;p->kind=2;p->features=3;return 0;}
static int send(void*u,const uint8_t*r,const uint8_t*l){Fake*f=u;memcpy(f->bytes[0],l,11);memcpy(f->bytes[1],r,11);f->sends++;return 0;}
static int led(void*u,uint8_t r,uint8_t g,uint8_t b){(void)u;(void)r;(void)g;(void)b;return 0;}
static int capture(void*u){return ((Fake*)u)->capture;}
static int begin(void*u,const char*t,const char*id){(void)u;(void)t;(void)id;return 1;}
static void end(void*u){(void)u;}
static void text(void*u,const char*t){(void)u;(void)t;}
static int button(void*u,const char*t){Fake*f=u;if(f->click&&!strcmp(t,f->click)){f->click=NULL;return 1;}return 0;}
static int checkbox(void*u,const char*t,int32_t*v){Fake*f=u;if(f->click&&!strcmp(t,f->click)){*v=!*v;f->click=NULL;return 1;}return 0;}
static void logline(void*u,const char*id,int level,const char*msg){(void)u;(void)level;printf("%s: %s\n",id?id:"host",msg);}
static void tick(modhost*h){h->pad_next_ms=0;modhost_frame(h);}
int main(int argc,char**argv){
 assert(argc==2);Fake f={0};modhost_config c={0};c.mods_dir=argv[1];c.cache_dir="dualsense-dll-test-cache";
 c.pad=(modhost_pad_backend){&f,info,send,led,capture};c.log=logline;
 c.ui.user=&f;c.ui.begin_panel=begin;c.ui.end_panel=end;c.ui.text=text;c.ui.button=button;c.ui.checkbox=checkbox;
 modhost*h=modhost_create(&c);assert(modhost_scan(h)==1);
 mh_entry*e=mh_find(h,"bbmod.dualsense");assert(e&&e->state==MODHOST_ACTIVE);
 tick(h);assert(f.bytes[1][0]==1&&f.bytes[1][2]==30);
 f.capture=1;tick(h);assert(f.bytes[1][0]==5);
 f.click="Test L2 buzz";modhost_draw_panels(h);assert(!f.click);tick(h);assert(f.bytes[0][0]==5);
 f.capture=0;tick(h);assert(f.bytes[0][0]==6);
 f.capture=1;tick(h);assert(f.bytes[0][0]==5);f.capture=0;tick(h);assert(f.bytes[0][0]==5);
 assert(modhost_reload_now(h,"bbmod.dualsense")==0);assert(f.bytes[1][0]==5);tick(h);assert(f.bytes[1][0]==1);
 f.click="Reset triggers and disable";modhost_draw_panels(h);tick(h);assert(f.bytes[1][0]==5&&!h->trigger_owner[1]);
 assert(modhost_reload_now(h,"bbmod.dualsense")==0);tick(h);assert(f.bytes[1][0]==5);
 f.click="Enabled";modhost_draw_panels(h);tick(h);assert(f.bytes[1][0]==1);
 mh_mod_faulted(e->inst,"simulated callback fault");tick(h);assert(f.bytes[1][0]==5&&!h->trigger_owner[1]);
 assert(modhost_load_now(h,"bbmod.dualsense")==0);tick(h);assert(f.bytes[1][0]==1);
 assert(modhost_unload_now(h,"bbmod.dualsense")==0);assert(f.bytes[1][0]==5);
 char config_path[2048];snprintf(config_path,sizeof config_path,"%s/../config/bbmod.dualsense.toml",argv[1]);
 FILE*config_file=fopen(config_path,"wb");assert(config_file);fputs("[general]\nintensity = \"0.5\"\n",config_file);fclose(config_file);
 assert(modhost_load_now(h,"bbmod.dualsense")==0);tick(h);assert(f.bytes[1][2]==15);
 assert(modhost_unload_now(h,"bbmod.dualsense")==0);assert(remove(config_path)==0);
 modhost_destroy(h);puts("real DLL load, UI previews/reset, reload, simulated fault and unload: OK");return 0;
}
