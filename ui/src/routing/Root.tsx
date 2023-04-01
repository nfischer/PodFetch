import {useEffect} from "react";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {Header} from "../components/Header";
import {SideBar} from "../components/SideBar";
import {Outlet, useNavigate} from "react-router-dom";
import {AudioComponents} from "../components/AudioComponents";
import {Search} from "../components/Search";
import axios, {AxiosResponse} from "axios";
import {apiURL, configWSUrl} from "../utils/Utilities";
import {ConfigModel} from "../models/SysInfo";
import {setConfigModel, setLoginData} from "../store/CommonSlice";
import {Loading} from "../components/Loading";
import App from "../App";
import {AuthModel} from "../models/AuthModel";


export const Root = () => {
    const sideBarCollapsed = useAppSelector(state => state.common.sideBarCollapsed)
    const dispatch = useAppDispatch()
    const configModel = useAppSelector(state => state.common.configModel)
    const navigate = useNavigate()
    const auth = useAppSelector(state => state.common.loginData)

    useEffect(()=>{
        axios.get(apiURL+"/sys/config").then((v:AxiosResponse<ConfigModel>)=>{
            dispatch(setConfigModel(v.data))
        })
    },[])

    const extractLoginData = (auth_local: string)=>{
        const test = atob(auth_local)
        const res = test.split(":")

        auth_local && dispatch(setLoginData({password: res[1], username: res[0]}))
        axios.defaults.headers.common['Authorization'] = 'Basic ' + auth_local;
    }

    useEffect(()=>{
        if(configModel){
            if(configModel.basicAuth){
                const auth_local =  localStorage.getItem('auth')
                const auth_session = sessionStorage.getItem('auth')
                if(auth_local == undefined && auth_session === undefined && !auth){
                    navigate("/login")
                }
                else if (auth_local && !auth){
                    extractLoginData(auth_local)
                }
                else if (auth_session && !auth){
                    extractLoginData(auth_session)
                }
                else if (auth){
                    axios.defaults.headers.common['Authorization'] = 'Basic ' + btoa(auth.username + ":" + auth.password);
                }
            }
        }
    },[configModel])

    if(!configModel || (configModel.basicAuth && !auth)){
        return <Loading/>
    }

    configWSUrl(configModel.serverUrl)

    return <App>
        <div className="grid  grid-rows-[auto_1fr] h-full md:grid-cols-[300px_1fr]">
            <Header/>
            <SideBar/>
            <div
                className={`col-span-6 md:col-span-5 ${sideBarCollapsed ? 'xs:col-span-5' : 'hidden'} md:block w-full overflow-x-auto`}>
                <div className="grid grid-rows-[1fr_auto] h-full ">
                    <Outlet/>
                    <AudioComponents/>
                </div>
            </div>
        </div>
        <Search/>
    </App>
}
