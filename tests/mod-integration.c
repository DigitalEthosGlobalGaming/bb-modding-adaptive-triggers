#include "internal.h"
#include <assert.h>
#include <stdio.h>
typedef struct {int capture;const char *click;int sends,grips,stops;uint8_t depth;uint8_t bytes[2][11];} Fake;
static int info(void*u,bbm_pad_info*p){(void)u;memset(p,0,sizeof *p);p->device_id=1;p->kind=2;p->connection=1;p->features=3;p->l2_depth=((Fake*)u)->depth;return 0;}
static int send(void*u,const uint8_t*r,const uint8_t*l){Fake*f=u;memcpy(f->bytes[0],l,11);memcpy(f->bytes[1],r,11);f->sends++;return 0;}
static int led(void*u,uint8_t r,uint8_t g,uint8_t b){(void)u;(void)r;(void)g;(void)b;return 0;}
static int grip(void*u,const float*pcm,uint32_t n){Fake*f=u;if(n){assert(pcm&&n<=12000);f->grips++;}else f->stops++;return 0;}
static int capture(void*u){return ((Fake*)u)->capture;}
static int begin(void*u,const char*t,const char*id){(void)u;(void)t;(void)id;return 1;}
static int header(void*u,const char*t){(void)u;(void)t;return 1;}
static void end(void*u){(void)u;}
static void text(void*u,const char*t){(void)u;(void)t;}
static int button(void*u,const char*t){Fake*f=u;if(f->click&&!strcmp(t,f->click)){f->click=NULL;return 1;}return 0;}
static int checkbox(void*u,const char*t,int32_t*v){Fake*f=u;if(f->click&&!strcmp(t,f->click)){*v=!*v;f->click=NULL;return 1;}return 0;}
static void logline(void*u,const char*id,int level,const char*msg){(void)u;(void)level;printf("%s: %s\n",id?id:"host",msg);}
static void tick(modhost*h){h->pad_next_ms=0;modhost_frame(h);}
static uint32_t ammo, right_item=7000000, left_item=14000000;
static bbm_status equipment(bbm_handle m,bbm_equipment_snapshot*p){
 (void)m;memset(p,0,sizeof *p);p->size=sizeof *p;p->flags=1;p->generation=1;p->actor=2;p->character=3;
 p->selected_even=0;p->selected_odd=1;p->runtime_references[0]=10;p->runtime_references[1]=20;return BBM_OK;
}
static bbm_status inventory(bbm_handle m,uint64_t revision,uint32_t offset,uint32_t limit,bbm_inventory_page*p){
 (void)m;(void)limit;if(offset|| (revision&&revision!=1))return BBM_ERR_STATE;
 memset(p,0,sizeof *p);p->snapshot=(bbm_inventory_summary){.size=sizeof p->snapshot,.flags=1,.generation=1,.revision=1,.actor=2,.character=3,.slot_count=3,.item_count=3};p->count=3;
 p->entries[0]=(bbm_inventory_entry){.reference=10,.encoded_item_id=right_item,.quantity=1,.flags=1};
 p->entries[1]=(bbm_inventory_entry){.reference=20,.encoded_item_id=left_item,.quantity=1,.flags=1};
 p->entries[2]=(bbm_inventory_entry){.reference=30,.encoded_item_id=0x40000384,.quantity=ammo,.flags=1};return BBM_OK;
}
int main(int argc,char**argv){
 assert(argc==2);Fake f={0};modhost_config c={0};c.mods_dir=argv[1];c.cache_dir="dualsense-dll-test-cache";
 c.pad=(modhost_pad_backend){&f,info,send,led,capture,grip};c.log=logline;
 c.ui.user=&f;c.ui.begin_panel=begin;c.ui.end_panel=end;c.ui.text=text;c.ui.button=button;c.ui.checkbox=checkbox;c.ui.collapsing_header=header;
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
 h->api.equipment_snapshot=equipment;h->api.inventory_page=inventory;
 assert(modhost_load_now(h,"bbmod.dualsense")==0);tick(h);
 assert(f.bytes[0][0]==5&&f.bytes[1][2]==25); /* empty pistol + cleaver */
 ammo=5;f.depth=200;plat_sleep_ms(260);tick(h);assert(f.bytes[0][0]==1&&f.bytes[0][2]==85);assert(f.grips==1);
 ammo=0;plat_sleep_ms(260);tick(h);assert(f.bytes[0][0]==5);assert(f.stops==1&&!h->haptics_owner);
 right_item=5000000;left_item=19000000;plat_sleep_ms(260);tick(h);assert(f.bytes[0][2]==75&&f.bytes[1][2]==45);
 f.capture=1;tick(h);f.click="Test grip texture after closing F1";modhost_draw_panels(h);assert(!f.click);
 int before_grips=f.grips;tick(h);assert(f.grips==before_grips);
 f.capture=0;tick(h);assert(f.grips==before_grips+1 && h->haptics_owner);
 int before_stops=f.stops;tick(h);assert(f.stops==before_stops); /* preview is not immediately cancelled */
 f.capture=1;tick(h);assert(f.stops==before_stops+1 && !h->haptics_owner);
 f.click="Test grip texture after closing F1";modhost_draw_panels(h);f.capture=0;tick(h);assert(h->haptics_owner);
 assert(modhost_reload_now(h,"bbmod.dualsense")==0);assert(!h->haptics_owner);
 assert(modhost_unload_now(h,"bbmod.dualsense")==0);assert(f.bytes[0][0]==5&&f.bytes[1][0]==5);
 modhost_destroy(h);puts("real DLL load, UI previews/reset, reload, simulated fault and unload: OK");return 0;
}
