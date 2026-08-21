import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';
import { ActivatedRouteSnapshot, Router } from '@angular/router';

import { EMPTY, Observable, catchError, of } from 'rxjs';

import { IJobOffer } from '../job-offer.model';
import { JobOfferService } from '../service/job-offer.service';

const jobOfferResolve = (route: ActivatedRouteSnapshot): Observable<null | IJobOffer> => {
  const { id } = route.params;
  if (id) {
    const router = inject(Router);
    const service = inject(JobOfferService);
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

export default jobOfferResolve;
