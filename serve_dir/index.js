async function main(){
    for(var i =0; i<5; i++){
        put({x:"hello there", y:i}).then();
        console.log(i);
    }
    console.log("done");
}

async function put(value){
    fetch("index.js",  {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body:JSON.stringify(value),
    }).then(x=>{
        console.log(x);
    }).then();
}

main().then();