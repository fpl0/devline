import {test} from 'node:test';import assert from 'node:assert/strict';
import {mkdtempSync,rmSync} from 'node:fs';import {tmpdir} from 'node:os';import {join} from 'node:path';
import {buildApp} from '../src/app.mjs';
import {SignJWT,jwtVerify} from 'jose';
test('real API validates, persists, reads, and deletes tasks',async()=>{
 const directory=mkdtempSync(join(tmpdir(),'devline-node-sqlite-'));const database=join(directory,'tasks.sqlite');let app=await buildApp({database});
 try{
  assert.equal((await app.inject({method:'POST',url:'/tasks',payload:{title:''}})).statusCode,400);
  const created=await app.inject({method:'POST',url:'/tasks',payload:{title:'Build offline'}});assert.equal(created.statusCode,201);
  const task=created.json();assert.equal(task.title,'Build offline');await app.close();app=await buildApp({database});
  assert.deepEqual((await app.inject('/tasks')).json(),[task]);
  assert.equal((await app.inject({method:'DELETE',url:`/tasks/${task.id}`})).statusCode,204);
  assert.deepEqual((await app.inject('/tasks')).json(),[]);
 }finally{await app.close();rmSync(directory,{recursive:true,force:true})}
});
test('JWT signing and verification operate locally',async()=>{const key=crypto.getRandomValues(new Uint8Array(32));const jwt=await new SignJWT({scope:'offline'}).setProtectedHeader({alg:'HS256'}).setIssuedAt().setExpirationTime('1h').sign(key);const {payload}=await jwtVerify(jwt,key,{algorithms:['HS256']});assert.equal(payload.scope,'offline');await assert.rejects(()=>jwtVerify(jwt,crypto.getRandomValues(new Uint8Array(32)),{algorithms:['HS256']}));});
test('database, queue, object storage, and HTTP clients load without connecting',async()=>{
 const pg=await import('pg');assert.equal(typeof pg.Pool,'function');
 const mysql=await import('mysql2/promise');assert.equal(typeof mysql.createConnection,'function');
 const redis=await import('redis');const client=redis.createClient();assert.equal(client.isOpen,false);
 const amqp=await import('amqplib');assert.equal(typeof amqp.connect,'function');
 const nats=await import('@nats-io/transport-node');assert.equal(typeof nats.connect,'function');
 const {S3Client}=await import('@aws-sdk/client-s3');const s3=new S3Client({region:'local',endpoint:'http://127.0.0.1:8333',credentials:{accessKeyId:'example',secretAccessKey:'example'}});assert.ok(s3);s3.destroy();
 const http=await import('undici');assert.equal(typeof http.request,'function');
});
