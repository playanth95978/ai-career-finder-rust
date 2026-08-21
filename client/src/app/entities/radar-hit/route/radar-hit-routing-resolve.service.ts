import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';
import { ActivatedRouteSnapshot, Router } from '@angular/router';

import { EMPTY, Observable, catchError, of } from 'rxjs';

import { IRadarHit } from '../radar-hit.model';
import { RadarHitService } from '../service/radar-hit.service';

const radarHitResolve = (route: ActivatedRouteSnapshot): Observable<null | IRadarHit> => {
  const { id } = route.params;
  if (id) {
    const router = inject(Router);
    const service = inject(RadarHitService);
    return service.find(id).pipe(
      catchError((error: HttpErrorResponse) => {
        if (error.status === 404) {
          router.navigate(['404']);
        } else {
          router.navigate(['error']);
        }
        return EMPTY;
      }),
    );
  }

  return of(null);
};

export default radarHitResolve;
